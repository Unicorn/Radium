# Extension System API Reference

Complete API reference for the Radium extension system. This document covers all public types, functions, and modules available for integrating with the extension system.

## Table of Contents

- [Core Types](#core-types)
- [Extension Manager API](#extension-manager-api)
- [Discovery API](#discovery-api)
- [Marketplace API](#marketplace-api)
- [Signing API](#signing-api)
- [Integration Helpers](#integration-helpers)
- [Error Types](#error-types)

## Core Types

### ExtensionManifest

Represents an extension manifest file (`radium-extension.json`).

```rust
pub struct ExtensionManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub components: ExtensionComponents,
    pub dependencies: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

**Methods:**

- `load(path: &Path) -> Result<Self>` - Load manifest from file
- `from_json(json: &str) -> Result<Self>` - Parse manifest from JSON string
- `validate() -> Result<()>` - Validate manifest structure and content
- `to_json() -> String` - Serialize manifest to JSON

**Example:**

```rust
use radium_core::extensions::ExtensionManifest;
use std::path::Path;

let manifest = ExtensionManifest::load(Path::new("./my-extension/radium-extension.json"))?;
manifest.validate()?;
println!("Extension: {} v{}", manifest.name, manifest.version);
```

### ExtensionComponents

Defines component paths for an extension.

```rust
pub struct ExtensionComponents {
    pub prompts: Vec<String>,
    pub mcp_servers: Vec<String>,
    pub commands: Vec<String>,
    pub hooks: Vec<String>,
}
```

**Fields:**
- `prompts`: Glob patterns for prompt files (`.md`)
- `mcp_servers`: Paths to MCP server configs (`.json`)
- `commands`: Glob patterns for command files (`.toml`)
- `hooks`: Glob patterns for hook files (`.toml`)

### Extension

Represents an installed extension.

```rust
pub struct Extension {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub install_path: PathBuf,
    // ... internal fields
}
```

**Methods:**

- `prompts_dir() -> PathBuf` - Get prompts directory path
- `mcp_dir() -> PathBuf` - Get MCP directory path
- `commands_dir() -> PathBuf` - Get commands directory path
- `hooks_dir() -> PathBuf` - Get hooks directory path
- `get_mcp_paths() -> Result<Vec<PathBuf>>` - Get all MCP config file paths
- `get_hook_paths() -> Result<Vec<PathBuf>>` - Get all hook file paths

## Extension Manager API

### ExtensionManager

Main API for installing, uninstalling, and managing extensions.

```rust
pub struct ExtensionManager {
    // ... internal fields
}
```

**Construction:**

```rust
// Create with default extensions directory (~/.radium/extensions/)
let manager = ExtensionManager::new()?;

// Create with custom directory
let manager = ExtensionManager::with_directory(PathBuf::from("/custom/path"));
```

**Methods:**

#### `install(source: &Path, options: InstallOptions) -> Result<Extension>`

Install an extension from a local directory or archive.

**Arguments:**
- `source`: Path to extension directory or archive file
- `options`: Installation options

**Example:**

```rust
use radium_core::extensions::{ExtensionManager, InstallOptions};
use std::path::Path;

let manager = ExtensionManager::new()?;
let options = InstallOptions {
    overwrite: false,
    install_dependencies: true,
    validate_after_install: true,
};

let extension = manager.install(Path::new("./my-extension"), options)?;
println!("Installed: {}", extension.name);
```

#### `install_from_source(source: &str, options: InstallOptions) -> Result<Extension>`

Install from local directory, archive, or URL.

**Arguments:**
- `source`: Path, archive file, or URL
- `options`: Installation options

**Example:**

```rust
// From local directory
let ext = manager.install_from_source("./my-extension", options.clone())?;

// From archive
let ext = manager.install_from_source("./my-extension.tar.gz", options.clone())?;

// From URL
let ext = manager.install_from_source("https://example.com/ext.tar.gz", options)?;
```

#### `uninstall(name: &str) -> Result<()>`

Uninstall an extension by name.

**Example:**

```rust
manager.uninstall("my-extension")?;
```

#### `list() -> Result<Vec<Extension>>`

List all installed extensions.

**Example:**

```rust
let extensions = manager.list()?;
for ext in extensions {
    println!("{} v{}", ext.name, ext.version);
}
```

#### `get(name: &str) -> Result<Option<Extension>>`

Get a specific extension by name.

**Example:**

```rust
if let Some(ext) = manager.get("my-extension")? {
    println!("Found: {}", ext.name);
}
```

#### `update(name: &str, package_path: &Path, options: InstallOptions) -> Result<Extension>`

Update an extension to a new version.

**Example:**

```rust
let updated = manager.update("my-extension", Path::new("./new-version"), options)?;
```

### InstallOptions

Configuration for extension installation.

```rust
pub struct InstallOptions {
    pub overwrite: bool,
    pub install_dependencies: bool,
    pub validate_after_install: bool,
}
```

**Fields:**
- `overwrite`: Overwrite existing installation
- `install_dependencies`: Automatically install dependencies
- `validate_after_install`: Validate structure after installation

## Discovery API

### ExtensionDiscovery

Service for discovering and searching installed extensions.

```rust
pub struct ExtensionDiscovery {
    // ... internal fields
}
```

**Construction:**

```rust
// Default discovery (searches ~/.radium/extensions/)
let discovery = ExtensionDiscovery::new();

// Custom search paths
let options = DiscoveryOptions {
    search_paths: vec![
        PathBuf::from("~/.radium/extensions"),
        PathBuf::from(".radium/extensions"),
    ],
    validate_structure: true,
};
let discovery = ExtensionDiscovery::with_options(options);
```

**Methods:**

#### `discover_all() -> Result<Vec<Extension>>`

Discover all installed extensions.

**Example:**

```rust
let extensions = discovery.discover_all()?;
println!("Found {} extensions", extensions.len());
```

#### `get(name: &str) -> Result<Option<Extension>>`

Get a specific extension by name.

**Example:**

```rust
if let Some(ext) = discovery.get("my-extension")? {
    println!("Found: {}", ext.name);
}
```

#### `search(query: &str) -> Result<Vec<Extension>>`

Search extensions by name or description.

**Example:**

```rust
let results = discovery.search("github")?;
for ext in results {
    println!("Match: {} - {}", ext.name, ext.description);
}
```

### DiscoveryOptions

Configuration for extension discovery.

```rust
pub struct DiscoveryOptions {
    pub search_paths: Vec<PathBuf>,
    pub validate_structure: bool,
}
```

**Fields:**
- `search_paths`: Directories to search (empty = default)
- `validate_structure`: Validate extension structure during discovery

## Marketplace API

### MarketplaceClient

Client for interacting with the extension marketplace.

```rust
pub struct MarketplaceClient {
    // ... internal fields
}
```

**Construction:**

```rust
// Default client (uses RADIUM_MARKETPLACE_URL env var or default)
let client = MarketplaceClient::new()?;

// Custom URL
let client = MarketplaceClient::with_url("https://marketplace.example.com/api/v1".to_string())?;
```

**Methods:**

#### `search(query: &str) -> Result<Vec<MarketplaceExtension>>`

Search marketplace for extensions.

**Example:**

```rust
let results = client.search("code review")?;
for ext in results {
    println!("{} v{} - {}", ext.name, ext.version, ext.description);
}
```

#### `get_extension_info(name: &str) -> Result<Option<MarketplaceExtension>>`

Get extension information by name.

**Example:**

```rust
if let Some(ext) = client.get_extension_info("github-integration")? {
    println!("Download URL: {}", ext.download_url);
}
```

#### `browse() -> Result<Vec<MarketplaceExtension>>`

Browse popular extensions.

**Example:**

```rust
let popular = client.browse()?;
for ext in popular {
    println!("⭐ {} - {} downloads", ext.name, ext.download_count.unwrap_or(0));
}
```

### MarketplaceExtension

Marketplace extension metadata.

```rust
pub struct MarketplaceExtension {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub download_url: String,
    pub download_count: Option<u64>,
    pub rating: Option<f64>,
    pub tags: Vec<String>,
    pub manifest: Option<ExtensionManifest>,
}
```

### ExtensionPublisher

Publish extensions to the marketplace.

```rust
pub struct ExtensionPublisher {
    // ... internal fields
}
```

**Methods:**

#### `publish(extension_path: &Path, api_key: &str) -> Result<()>`

Publish an extension to the marketplace.

**Example:**

```rust
use radium_core::extensions::ExtensionPublisher;
use std::path::Path;

let publisher = ExtensionPublisher::new()?;
publisher.publish(Path::new("./my-extension"), "api-key-here")?;
```

## Signing API

### ExtensionSigner

Sign extensions with cryptographic signatures.

```rust
pub struct ExtensionSigner {
    // ... internal fields
}
```

**Construction:**

```rust
// Generate new keypair
let (signer, public_key) = ExtensionSigner::generate();

// Load from private key
let private_key_bytes = fs::read("private.key")?;
let signer = ExtensionSigner::from_private_key(&private_key_bytes)?;
```

**Methods:**

#### `sign_extension(extension_path: &Path) -> Result<PathBuf>`

Sign an extension package.

**Example:**

```rust
let signature_path = signer.sign_extension(Path::new("./my-extension"))?;
println!("Signature saved to: {}", signature_path.display());
```

### SignatureVerifier

Verify extension signatures.

```rust
pub struct SignatureVerifier {
    // ... internal fields
}
```

**Construction:**

```rust
// From public key bytes
let public_key_bytes = fs::read("public.key")?;
let verifier = SignatureVerifier::from_public_key(&public_key_bytes)?;
```

**Methods:**

#### `verify_extension(extension_path: &Path) -> Result<()>`

Verify an extension signature.

**Example:**

```rust
verifier.verify_extension(Path::new("./my-extension"))?;
println!("Signature verified!");
```

### TrustedKeysManager

Manage trusted signing keys.

```rust
pub struct TrustedKeysManager {
    // ... internal fields
}
```

**Methods:**

- `add_trusted_key(name: &str, public_key: &[u8]) -> Result<()>`
- `remove_trusted_key(name: &str) -> Result<()>`
- `list_trusted_keys() -> Result<Vec<(String, Vec<u8>)>>`
- `is_trusted(public_key: &[u8]) -> bool`

**Example:**

```rust
let manager = TrustedKeysManager::new()?;
manager.add_trusted_key("Publisher Name", &public_key_bytes)?;

if manager.is_trusted(&public_key_bytes) {
    println!("Key is trusted");
}
```

## Integration Helpers

Helper functions for integrating extensions into other systems.

### `get_all_extensions() -> Result<Vec<Extension>>`

Get all installed extensions.

**Example:**

```rust
use radium_core::extensions::get_all_extensions;

let extensions = get_all_extensions()?;
for ext in extensions {
    println!("{} v{}", ext.name, ext.version);
}
```

### `get_extension_prompt_dirs() -> Result<Vec<PathBuf>>`

Get all extension prompt directories.

**Example:**

```rust
use radium_core::extensions::get_extension_prompt_dirs;

let dirs = get_extension_prompt_dirs()?;
for dir in dirs {
    // Load prompts from directory
    load_prompts_from_dir(&dir)?;
}
```

### `get_extension_command_dirs() -> Result<Vec<PathBuf>>`

Get all extension command directories.

**Example:**

```rust
use radium_core::extensions::get_extension_command_dirs;

let dirs = get_extension_command_dirs()?;
for dir in dirs {
    // Register commands from directory
    register_commands_from_dir(&dir)?;
}
```

### `get_extension_mcp_configs() -> Result<Vec<PathBuf>>`

Get all extension MCP server configuration paths.

**Example:**

```rust
use radium_core::extensions::get_extension_mcp_configs;

let configs = get_extension_mcp_configs()?;
for config_path in configs {
    // Load MCP server config
    load_mcp_config(&config_path)?;
}
```

## Versioning API

### VersionComparator

Compare and validate semantic versions.

```rust
pub struct VersionComparator;
```

**Methods:**

- `parse(version_str: &str) -> Result<Version>` - Parse version string
- `compare(v1: &str, v2: &str) -> Result<Ordering>` - Compare two versions
- `is_compatible_with(version: &str, constraint: &str) -> Result<bool>` - Check version constraint
- `is_newer(new_version: &str, old_version: &str) -> Result<bool>` - Check if newer

**Example:**

```rust
use radium_core::extensions::VersionComparator;
use std::cmp::Ordering;

let ordering = VersionComparator::compare("2.0.0", "1.0.0")?;
assert_eq!(ordering, Ordering::Greater);

let compatible = VersionComparator::is_compatible_with("1.2.0", "^1.0.0")?;
assert!(compatible);
```

### UpdateChecker

Check for extension updates.

```rust
pub struct UpdateChecker;
```

**Methods:**

- `check_for_update(extension: &Extension, new_version: &str) -> Result<bool>` - Check if update available
- `validate_constraint(new_version: &str, constraint: Option<&str>) -> Result<bool>` - Validate version constraint

**Example:**

```rust
use radium_core::extensions::{UpdateChecker, Extension};

let has_update = UpdateChecker::check_for_update(&extension, "2.0.0")?;
if has_update {
    println!("Update available!");
}
```

## Error Types

### ExtensionError

Unified error type for all extension operations.

```rust
pub enum ExtensionError {
    Manifest(ExtensionManifestError),
    Structure(ExtensionStructureError),
    Discovery(ExtensionDiscoveryError),
    Installer(ExtensionInstallerError),
}
```

### ExtensionManifestError

Manifest parsing and validation errors.

```rust
pub enum ExtensionManifestError {
    Io(std::io::Error),
    JsonParse(serde_json::Error),
    InvalidFormat(String),
    MissingField(String),
    InvalidVersion(String),
    InvalidComponentPath(String),
    NotFound(String),
}
```

### ExtensionInstallerError

Installation and management errors.

```rust
pub enum ExtensionInstallerError {
    Io(std::io::Error),
    Manifest(ExtensionManifestError),
    Structure(ExtensionStructureError),
    Discovery(ExtensionDiscoveryError),
    AlreadyInstalled(String),
    NotFound(String),
    Dependency(String),
    InvalidFormat(String),
    Conflict(String),
    Validation(ExtensionValidationError),
    #[cfg(feature = "workflow")]
    ConflictDetection(ConflictError),
}
```

### ExtensionDiscoveryError

Discovery and search errors.

```rust
pub enum ExtensionDiscoveryError {
    Io(std::io::Error),
    Manifest(ExtensionManifestError),
    Structure(ExtensionStructureError),
    NotFound(String),
}
```

### MarketplaceError

Marketplace operation errors.

```rust
pub enum MarketplaceError {
    Http(reqwest::Error),
    JsonParse(serde_json::Error),
    InvalidResponse(String),
    Timeout,
}
```

### SigningError

Signing and verification errors.

```rust
pub enum SigningError {
    Io(std::io::Error),
    InvalidKey(String),
    VerificationFailed(String),
    SignatureNotFound(String),
    Manifest(String),
}
```

### VersioningError

Version comparison and validation errors.

```rust
pub enum VersioningError {
    InvalidVersion(String),
    InvalidConstraint(String),
    Comparison(String),
}
```

## Error Handling Patterns

All extension APIs use `Result<T, E>` types for error handling. Use the `?` operator for error propagation:

```rust
use radium_core::extensions::{ExtensionManager, InstallOptions, ExtensionError};
use std::path::Path;

fn install_extension() -> Result<(), ExtensionError> {
    let manager = ExtensionManager::new()?;
    let options = InstallOptions::default();
    let extension = manager.install(Path::new("./my-extension"), options)?;
    println!("Installed: {}", extension.name);
    Ok(())
}
```

For specific error handling:

```rust
use radium_core::extensions::{ExtensionManager, ExtensionInstallerError};

match manager.install(path, options) {
    Ok(ext) => println!("Installed: {}", ext.name),
    Err(ExtensionInstallerError::AlreadyInstalled(name)) => {
        println!("Extension {} already installed", name);
    }
    Err(ExtensionInstallerError::Dependency(msg)) => {
        eprintln!("Dependency error: {}", msg);
    }
    Err(e) => eprintln!("Installation failed: {}", e),
}
```

## Feature Flags

Some modules are feature-gated:

- `workflow`: Enables conflict detection (`conflict.rs` module)

**Example:**

```rust
#[cfg(feature = "workflow")]
use radium_core::extensions::conflict::ConflictDetector;
```

## Next Steps

- [Integration Guide](integration-guide.md) - Learn how to integrate the extension system
- [Architecture](architecture.md) - Understand the system architecture
- [User Guide](user-guide.md) - User-facing documentation

